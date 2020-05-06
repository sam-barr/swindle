int n = 99;

while n >= 1 {
    string bottles = if n == 1 { " bottle "; } else { " bottles "; };
    writeln $ n bottles "of beer on the wall";
    writeln $ n bottles "of beer";
    writeln "Take on down, pass it around";
    bottles = if n-1 == 1 { " bottle "; } else { " bottles "; };
    writeln $ (n-1) bottles "of beer on the wall";

    n = n - 1;
    writeln "";
};
