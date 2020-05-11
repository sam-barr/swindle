int x = 1;

while x <= 100 {
    bool three = x % 3 == 0;
    bool five = x % 5 == 0;

    if three and five {
        writeln "fizzbuzz";
    } elif three {
        writeln "fizz";
    } elif five {
        writeln "buzz";
    } else {
        writeln x;
    };

    x = x + 1;
    unit;
};
