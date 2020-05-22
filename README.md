# swindle
A statically typed, expression based imperative language which compiles to LLVM.

    cargo run source.sw        # prints LLVM-IR to stderr

# FizzBuzz
The following is fizzbuzz written in "idiomatic" swindle:

```
int x = 1;

while x <= 100 {
    bool three = x % 3 == 0;
    bool five  = x % 5 == 0;

    if three or five {
        @writeln(
            if three { "fizz"; } else { ""; },
            if five  { "buzz"; } else { ""; }
        );
    } else {
        @writeln(x);
    };

    x = x + 1;
    unit;
};
```
