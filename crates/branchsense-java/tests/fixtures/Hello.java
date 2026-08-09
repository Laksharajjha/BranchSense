package fixtures;

public final class Hello {
    private final String message;

    public Hello(String message) {
        this.message = message;
    }

    public String message() {
        return message;
    }
}
