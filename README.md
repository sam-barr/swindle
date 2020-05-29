# swindle
A statically typed, expression based imperative language which compiles to LLVM.

    cargo run source.sw        # prints LLVM-IR to stderr

# FizzBuzz
The following is fizzbuzz written in "idiomatic" swindle:

```
for int i = 1; i <= 100; i = i + 1 {
    bool three = i % 3 == 0;
    bool five  = i % 5 == 0;

    if three or five {
        @writeln(
            if three { "fizz"; } else { ""; },
            if five  { "buzz"; } else { ""; }
        );
    } else {
        @writeln(i);
    };
};
```
