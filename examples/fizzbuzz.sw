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
