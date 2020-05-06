int x = (if true { writeln "hi"; 1; } else { 2; }) + (if false { 1;} else { writeln "bye"; 2;});
