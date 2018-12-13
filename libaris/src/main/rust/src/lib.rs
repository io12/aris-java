#[macro_use] extern crate nom;
extern crate jni;

pub mod parser;

use jni::JNIEnv;
use jni::strings::JavaStr;
use jni::objects::{JClass, JString, JValue, JObject};
use jni::sys::{jobject, jstring};

fn jobject_to_string(env: &JNIEnv, obj: JObject) -> jni::errors::Result<String> {
    Ok(String::from(env.get_string(JString::from(obj))?))
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_edu_rpi_aris_ast_Expression_toStringViaRust(env: JNIEnv, obj: JObject) -> jstring {
    (|| -> jni::errors::Result<jstring> {
        let expr = jobject_to_expr(&env, obj);
        //println!("toStringViaRust, expr: {:?}", expr);
        Ok(env.new_string(format!("{:?}", expr?))?.into_inner())
    })().unwrap_or(std::ptr::null_mut())
}

pub fn jobject_to_expr(env: &JNIEnv, obj: JObject) -> jni::errors::Result<Expr> {
    let cls = env.call_method(obj, "getClass", "()Ljava/lang/Class;", &[])?.l()?;
    let name = String::from(env.get_string(JString::from(env.call_method(cls, "getName", "()Ljava/lang/String;", &[])?.l()?))?);
    use expression_builders::*;
    let handle_binop = |symbol: BSymbol| -> jni::errors::Result<Expr> {
        let left = env.get_field(obj, "l", "Ledu/rpi/aris/ast/Expression;")?.l()?;
        let right = env.get_field(obj, "r", "Ledu/rpi/aris/ast/Expression;")?.l()?;
        Ok(binop(symbol, jobject_to_expr(env, left)?, jobject_to_expr(env, right)?))
    };
    let handle_abe = |symbol: ASymbol| -> jni::errors::Result<Expr> {
        let exprs_ = env.get_field(obj, "exprs", "Ljava/util/ArrayList;")?.l()?;
        let iter = env.call_method(exprs_, "iterator", "()Ljava/util/Iterator;", &[])?.l()?;
        let mut exprs = vec![];
        while env.call_method(iter, "hasNext", "()Z", &[])?.z()? {
            let expr = jobject_to_expr(env, env.call_method(iter, "next", "()Ljava/lang/Object;", &[])?.l()?)?;
            exprs.push(expr);
        }
        Ok(Expr::AssocBinop { symbol, exprs })
    };
    let handle_quantifier = |symbol: QSymbol| -> jni::errors::Result<Expr> {
        let name = jobject_to_string(env, env.get_field(obj, "boundvar", "Ljava/lang/String;")?.l()?)?;
        let body = env.get_field(obj, "body", "Ledu/rpi/aris/ast/Expression;")?.l()?;
        Ok(Expr::Quantifier { symbol, name, body: Box::new(jobject_to_expr(env, body)?) })
    };
    match &*name {
        "edu.rpi.aris.ast.Expression$NotExpression" => {
            let operand = env.get_field(obj, "operand", "Ledu/rpi/aris/ast/Expression;")?.l()?;
            Ok(notexp(jobject_to_expr(env, operand)?))
        },
        "edu.rpi.aris.ast.Expression$ImplicationExpression" => handle_binop(BSymbol::Implies),
        "edu.rpi.aris.ast.Expression$AddExpression" => handle_binop(BSymbol::Plus),
        "edu.rpi.aris.ast.Expression$MultExpression" => handle_binop(BSymbol::Mult),
        "edu.rpi.aris.ast.Expression$AndExpression" => handle_abe(ASymbol::And),
        "edu.rpi.aris.ast.Expression$OrExpression" => handle_abe(ASymbol::Or),
        "edu.rpi.aris.ast.Expression$BiconExpression" => handle_abe(ASymbol::Bicon),
        "edu.rpi.aris.ast.Expression$ForallExpression" => handle_quantifier(QSymbol::Forall),
        "edu.rpi.aris.ast.Expression$ExistsExpression" => handle_quantifier(QSymbol::Exists),
        "edu.rpi.aris.ast.Expression$BottomExpression" => Ok(Expr::Bottom),
        "edu.rpi.aris.ast.Expression$PredicateExpression" => {
            let name = jobject_to_string(env, env.get_field(obj, "name", "Ljava/lang/String;")?.l()?)?;
            let args_ = env.get_field(obj, "args", "Ljava/util/List;")?.l()?;
            let iter = env.call_method(args_, "iterator", "()Ljava/util/Iterator;", &[])?.l()?;
            let mut args = vec![];
            while env.call_method(iter, "hasNext", "()Z", &[])?.z()? {
                let arg = jobject_to_string(env, env.call_method(iter, "next", "()Ljava/lang/Object;", &[])?.l()?)?;
                args.push(arg);
            }
            Ok(Expr::Predicate { name, args })
        },
        _ => Err(jni::errors::Error::from_kind(jni::errors::ErrorKind::Msg(format!("jobject_to_expr: unknown class {}", name)))),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_edu_rpi_aris_ast_Expression_parseViaRust(env: JNIEnv, _cls: JClass, e: JString) -> jobject {
    (|| -> jni::errors::Result<jobject> {
        if let Ok(e) = JavaStr::from_env(&env, e)?.to_str() {
            //println!("received {:?}", e);
            let e = format!("{}\n", e);
            let parsed = parser::main(&e);
            //println!("parse: {:?}", parsed);
            if let Ok(("", expr)) = parsed {
                let r = expr_to_jobject(&env, expr)?;
                Ok(r.into_inner())
            } else {
                Ok(std::ptr::null_mut())
            }
        } else {
            Ok(std::ptr::null_mut())
        }
    })().unwrap_or(std::ptr::null_mut())
}

pub fn expr_to_jobject<'a>(env: &'a JNIEnv, e: Expr) -> jni::errors::Result<JObject<'a>> {
    let obj = env.new_object(e.get_class(), "()V", &[])?;
    let jv = |s: &str| -> jni::errors::Result<JValue> { Ok(JObject::from(env.new_string(s)?).into()) };
    let rec = |e: Expr| -> jni::errors::Result<JValue> { Ok(JObject::from(expr_to_jobject(env, e)?).into()) };
    match e {
        Expr::Bottom => (),
        Expr::Predicate { name, args } => {
            env.set_field(obj, "name", "Ljava/lang/String;", jv(&name)?)?;
            let list = env.get_field(obj, "args", "Ljava/util/List;")?.l()?;
            for arg in args {
                env.call_method(list, "add", "(Ljava/lang/Object;)Z", &[jv(&arg)?])?;
            }
        },
        Expr::Unop { symbol: _, operand } => {
            env.set_field(obj, "operand", "Ledu/rpi/aris/ast/Expression;", rec(*operand)?)?;
        },
        Expr::Binop { symbol: _, left, right } => {
            env.set_field(obj, "l", "Ledu/rpi/aris/ast/Expression;", rec(*left)?)?;
            env.set_field(obj, "r", "Ledu/rpi/aris/ast/Expression;", rec(*right)?)?;
        },
        Expr::AssocBinop { symbol: _, exprs } => {
            let list = env.get_field(obj, "exprs", "Ljava/util/ArrayList;")?.l()?;
            for expr in exprs {
                env.call_method(list, "add", "(Ljava/lang/Object;)Z", &[rec(expr)?])?;
            }
        }
        Expr::Quantifier { symbol: _, name, body } => {
            env.set_field(obj, "boundvar", "Ljava/lang/String;", jv(&name)?)?;
            env.set_field(obj, "body", "Ledu/rpi/aris/ast/Expression;", rec(*body)?)?;
        }
    }
    Ok(obj)
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum USymbol { Not }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BSymbol { Implies, Plus, Mult }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ASymbol { And, Or, Bicon }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QSymbol { Forall, Exists }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Bottom,
    Predicate { name: String, args: Vec<String> },
    Unop { symbol: USymbol, operand: Box<Expr> },
    Binop { symbol: BSymbol, left: Box<Expr>, right: Box<Expr> },
    AssocBinop { symbol: ASymbol, exprs: Vec<Expr> },
    Quantifier { symbol: QSymbol, name: String, body: Box<Expr> },
}

trait HasClass {
    fn get_class(&self) -> &'static str;
}
impl HasClass for USymbol {
    fn get_class(&self) -> &'static str {
        match self {
            USymbol::Not => "Ledu/rpi/aris/ast/Expression$NotExpression;",
        }
    }
}
impl HasClass for BSymbol {
    fn get_class(&self) -> &'static str {
        match self {
            BSymbol::Implies => "Ledu/rpi/aris/ast/Expression$ImplicationExpression;",
            BSymbol::Plus => "Ledu/rpi/aris/ast/Expression$AddExpression;",
            BSymbol::Mult => "Ledu/rpi/aris/ast/Expression$MultExpression;",
        }
    }
}
impl HasClass for ASymbol {
    fn get_class(&self) -> &'static str {
        match self {
            ASymbol::And => "Ledu/rpi/aris/ast/Expression$AndExpression;",
            ASymbol::Or => "Ledu/rpi/aris/ast/Expression$OrExpression;",
            ASymbol::Bicon => "Ledu/rpi/aris/ast/Expression$BiconExpression;",
        }
    }
}
impl HasClass for QSymbol {
    fn get_class(&self) -> &'static str {
        match self {
            QSymbol::Forall => "Ledu/rpi/aris/ast/Expression$ForallExpression;",
            QSymbol::Exists => "Ledu/rpi/aris/ast/Expression$ExistsExpression;",
        }
    }
}
impl HasClass for Expr {
    fn get_class(&self) -> &'static str {
        match self {
            Expr::Bottom => "Ledu/rpi/aris/ast/Expression$BottomExpression;",
            Expr::Predicate { .. } => "Ledu/rpi/aris/ast/Expression$PredicateExpression;",
            Expr::Unop { symbol, .. } => symbol.get_class(),
            Expr::Binop { symbol, .. } => symbol.get_class(),
            Expr::AssocBinop { symbol, .. } => symbol.get_class(),
            Expr::Quantifier { symbol, .. } => symbol.get_class(),
        }
    }
}

pub mod expression_builders {
    use super::{Expr, USymbol, BSymbol, ASymbol, QSymbol};
    pub fn predicate(name: &str, args: &[&str]) -> Expr { Expr::Predicate { name: name.into(), args: args.iter().map(|&x| x.into()).collect() } }
    pub fn notexp(expr: Expr) -> Expr { Expr::Unop { symbol: USymbol::Not, operand: Box::new(expr) } }
    pub fn binop(symbol: BSymbol, l: Expr, r: Expr) -> Expr { Expr::Binop { symbol, left: Box::new(l), right: Box::new(r) } }
    pub fn assocbinop(symbol: ASymbol, exprs: &[Expr]) -> Expr { Expr::AssocBinop { symbol, exprs: exprs.iter().cloned().collect() } }
    pub fn forall(name: &str, body: Expr) -> Expr { Expr::Quantifier { symbol: QSymbol::Forall, name: name.into(), body: Box::new(body) } }
    pub fn exists(name: &str, body: Expr) -> Expr { Expr::Quantifier { symbol: QSymbol::Exists, name: name.into(), body: Box::new(body) } }
}

#[cfg(test)]
mod tests {
}