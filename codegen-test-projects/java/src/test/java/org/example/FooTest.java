package org.example;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class FooTest {
    @Test
    void testSomething() {
        Foo foo = new Foo();

        StructWithFields data = new StructWithFields(1, "foo", true);

        // TODO: Ser/de round trip test.
    }
}
