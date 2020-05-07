int n = 0;
int num_thing = 17;

while true {
    int i = 0;

    while true {
        if i > n {
            break;
        };
        writeln i;
        i = i + 1;
    };

    if n > 5 {
        break;
    };
    writeln "~~~~~~~~~~";
    n = n + 1;
};
