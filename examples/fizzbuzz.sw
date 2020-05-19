int x = 1;

while x <= 100 {
    bool three = x % 3 == 0;
    bool five  = x % 5 == 0;

    if three or five {
        string fizz = if three { "fizz"; } else { ""; };
        string buzz = if five  { "buzz"; } else { ""; };
        write fizz;
        writeln buzz;
    } else {
        writeln x;
    };

    x = x + 1;
};
