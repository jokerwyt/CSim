# Cache simulator
The cache lab part A of CMU 15-213 (CSAPP), in Rust.

The usage and behaviour is nearly the same to the `csim-ref` in cachelab-handout, and it could pass `test-csim`.

# Fast-evaluation
For part B, `Valgrind` makes your transpone much slower. I mixed up my Rust `csim` and some C files to circumvent it, and provide a faster evaluation for cache misses.

I have to mention that **there will be some little error** (observed within 10) from the official `test-trans`, because I am not really tracking all memory access.

## How to use

1. put your transpose function into `fast-evalution/trans.c`.
2. modify every place in your code where you **ACCESS** array A or B, no matter they are Read or Write. For example:
```cpp
// B[i + x][j + y] = A[j + y][i + x];
_(B[i + x][j + y]) = _(A[j + y][i + x]);

// B[i + x][j + y] = r1;
_(B[i + x][j + y]) = r1;

// r1 = A[j + y][i + x];
r1 = _(A[j + y][i + x]);

```
3. run `make` under `fast-evalution`.
4. run `LD_LIBRARY_PATH=. ./fast-evalution/tracegen -M 32 -N 32`. If everything goes smoothly you can see the evaluation result like `miss: 340`. Adjust the size of matrix at your will.
You can also run `export LD_LIBRARY_PATH=.` in advance **every time you open a new terminal**, to replace the cmd with `./fast-evalution/tracegen -M 32 -N 32`.

5. It was recommended to overwrite `trans.c` in your handout with my version, so that you can copy and compile your code in both workspaces just with one line different. See comments in my `trans.c` for more details.


In fact, with C code of your `csim`, you can do whatever you want with no effort. Try to do it yourself.
