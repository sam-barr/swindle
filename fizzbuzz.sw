int x = 1;

while x <= 100 {
    bool three = x % 3 == 0;
    bool five = x % 5 == 0;

    string fizz = if three { "fizz"; } else { ""; };
    string buzz = if five  { "buzz"; } else { ""; };
    string n = if three or five { ""; } else { $ x; };
    writeln $ fizz buzz n;

    x = x + 1;
};
