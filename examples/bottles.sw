int n = 99;

while n >= 1 {
    string bottles = " bottle" + if n == 1 { ""; } else { "s"; } + " of beer";
    write   n;
    writeln bottles;

    write   n;
    write   bottles;
    writeln " of beer";

    n = n - 1;
    bottles = " bottle" + if n == 1 { ""; } else { "s"; } + " of beer on the wall\n";
    writeln "Take on down, pass it around";
    write   n;
    writeln bottles;
};
