package edu.rpi.aris.net.message;

import edu.rpi.aris.net.User;

import java.sql.Connection;

public class ErrorMsg extends Message {

    private final ErrorType errorType;
    private final String errorMsg;

    public ErrorMsg(String msg) {
        this(null, msg);
    }

    // DO NOT REMOVE!! Default constructor is required for gson deserialization
    private ErrorMsg() {
        errorMsg = null;
        errorType = null;
    }

    public ErrorMsg(ErrorType error) {
        this(error, null);
    }

    public ErrorMsg(ErrorType error, String msg) {
        errorType = error == null ? ErrorType.UNKNOWN_ERROR : error;
        errorMsg = msg;
    }

    public String getErrorMsg() {
        return errorMsg;
    }

    public ErrorType getErrorType() {
        return errorType;
    }

    @Override
    public ErrorType processMessage(Connection connection, User user) {
        return errorType;
    }

    @Override
    public MessageType getMessageType() {
        return MessageType.ERROR;
    }

    @Override
    public boolean checkValid() {
        return true;
    }

    @Override
    public String toString() {
        return "Error Message:\n" +
                "\tError Type: " + (errorType == null ? null : errorType.name()) + "\n" +
                "\tError Message: " + errorMsg;
    }
}