typedef unsigned int u32;
typedef unsigned long long u64;
extern void _C_interface_init_cache_manager(u32 set_bits, u32 set_size, u32 block_bits, u32 verbose);
// return value shows the operation result.
// 0 hit
// 1 miss but no eviction
// 2 miss and eviction 
extern int  _C_interface_access(u64 address);
extern u32  _C_interface_get_miss();

