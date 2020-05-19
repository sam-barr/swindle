int x = 1;

while x <= 100 {
    bool three = x % 3 == 0;
    bool five  = x % 5 == 0;

    if three or five {
        write   if three { "fizz"; } else { ""; };
        writeln if five  { "buzz"; } else { ""; };
    } else {
        writeln x;
    };

    x = x + 1;
};
