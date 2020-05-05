int x = 1;

while x <= 100 {
    bool three = x % 3 == 0;
    bool five = x % 5 == 0;

    write x;
    write ": ";
    if three and five {
        writeln "FizzBuzz";
    } elif three {
        writeln "Fizz";
    } elif five {
        writeln "Buzz";
    } else {
        writeln x;
    };

    x = x + 1;
};
