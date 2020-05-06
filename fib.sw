int n = 50;

int prev = 1;
int curr = 1;

int i = 0;
while i < n-2 {
    writeln curr;
    int temp = curr;
    curr = prev + curr;
    prev = temp;
    i = i + 1;
};

writeln curr;
