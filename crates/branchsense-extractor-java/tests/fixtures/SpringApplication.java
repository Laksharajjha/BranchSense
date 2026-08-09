package com.example.billing;

import java.util.List;
import org.springframework.stereotype.Service;

@Service
public class PaymentService<T extends Payment> implements Auditable {
    private final Repository repository;

    public PaymentService(Repository repository) {
        this.repository = repository;
    }

    /** Processes one payment. */
    @Override
    public List<T> process(T payment) {
        validate(payment);
        return repository.saveAll(List.of(payment));
    }

    private void validate(T payment) {}

    public interface Repository {
        List<T> saveAll(List<T> payments);
    }

    public static class Result {
        private final String status;
    }
}

interface Auditable {
    void audit();
}
