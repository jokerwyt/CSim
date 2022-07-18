#include <stdio.h>
#include "cachelab.h"
#include "contracts.h"
#include <string.h>





int is_transpose(int M, int N, int A[N][M], int B[M][N]);


// you just need to comment this define line.
// and this file will become a normal `trans.c`, 
// which behaves exactly the same with offical.
#define FAST_EVALUATION 

#ifdef FAST_EVALUATION
#include "csim.h"
#endif

#define _(X) *load(&(X))
int* load(int *ptr) {
#ifdef FAST_EVALUATION
    _C_interface_access((u64)ptr);  
#endif
    return ptr;
}

char transpose_submit_desc[] = "Transpose submission";
void transpose_submit(int M, int N, int A[N][M], int B[M][N])
{
    REQUIRES(M > 0);
    REQUIRES(N > 0);
#ifdef FAST_EVALUATION
    _C_interface_init_cache_manager(5, 1, 5, 0);
#endif
    ////////////////// put your transpose function below
    
    //Offical test-trans result: hits:1710, misses:343, evictions:311
    REQUIRES(M == N);
    const int block_size = 8;
    for(int i = 0; i < N; i += block_size) {
        for(int j = 0; j < M; j += block_size) {
            for(int x = 0; x < block_size; x++) {
                for(int y = 0; y < block_size; y++) {
                    _(B[i + x][j + y]) = _(A[j + y][i + x]);
                }
            }
        }
    }

    ////////////////// put your transpose function above

#ifdef FAST_EVALUATION
    printf("misses: %u\n", _C_interface_get_miss());
#endif
    ENSURES(is_transpose(M, N, A, B));
}

void registerFunctions()
{
    /* Register your solution function */
    registerTransFunction(transpose_submit, transpose_submit_desc);
}

/*
 * is_transpose - This helper function checks if B is the transpose of
 *     A. You can check the correctness of your transpose by calling
 *     it before returning from the transpose function.
 */
int is_transpose(int M, int N, int A[N][M], int B[M][N])
{
    int i, j;

    for (i = 0; i < N; i++) {
        for (j = 0; j < M; ++j) {
            if (A[i][j] != B[j][i]) {
                return 0;
            }
        }
    }
    return 1;
}

