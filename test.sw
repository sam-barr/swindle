int n = 0;

while true {
    if n == 4 {
        n = n + 1;
        continue;
    } elif n == 8 {
        break;
    };

    writeln n;
    n = n + 1;
};

writeln "done";
