package edu.rpi.aris.rules;

import edu.rpi.aris.TestUtil;
import org.junit.Test;

import java.text.ParseException;

public class DisjunctiveSyllogismTest {

    private String[][] premise = new String[][]{{"A → B", "A"}, {"(A ∧ B) → C", "A ∧ B"}, {"A → (B ∧ C)", "A"}, {"A ∧ B", "A"}, {"A → B", "B"}};
    private String[] conc = new String[]{"A", "B", "C", "A ∧ B", "B ∧ C"};
    private int[][] valid = new int[][]{};

    @Test
    public void test() throws ParseException {
        TestUtil.validateClaims(premise, conc, valid, RuleList.DISJUNCTIVE_SYLLOGISM);
    }

}