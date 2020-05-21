int n = 99;

while n >= 1 {
    string bottles = " bottle" + if n == 1 { ""; } else { "s"; };
    @writeln(n, bottles, " of beer on the wall");
    @writeln(n, bottles, " of beer");

    @writeln("Take on down, pass it around");
    n = n - 1;
    bottles = " bottle" + if n == 1 { ""; } else { "s"; };
    @writeln(n, bottles, " of beer on the wall");

    if n != 0 { @writeln(); };
};
