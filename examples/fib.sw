int n = 50;

writeln {
    int prev = 1;
    int curr = 1;
    int temp = 1;

    while n - 2 > 0 {
        n    = n - 1;
        temp = prev;
        prev = curr;
        curr = temp + curr;
    } else { 1; };
};
