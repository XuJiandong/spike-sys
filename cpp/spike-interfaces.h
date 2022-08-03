#ifndef __SPIKE_INTERFCES_H__
#define __SPIKE_INTERFCES_H__

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * vlen: Vector register size in bits, set it 128 by default
 * elen: Vector element size in bits, set it 64 by default
 * mem_size: if 0, disable memory; otherwise, set memory available in range [4096, 4096 + mem_size]
 */
uint64_t rvv_new_processor(uint32_t vlen, uint32_t elen, uint64_t mem_size);
int32_t rvv_execute(uint64_t processor, uint64_t instruction);
/**
 *  offset: the offset in register file in bytes
 *
 *  return: 0, success; otherwise, failed. Same to other functions
 */
int32_t rvv_get_vreg(uint64_t processor, uint64_t offset, uint8_t *mem, uint64_t mem_size);
int32_t rvv_set_vreg(uint64_t processor, uint64_t offset, uint8_t *mem, uint64_t mem_size);

int32_t rvv_get_xreg(uint64_t processor, uint64_t index, uint64_t *content);
int32_t rvv_set_xreg(uint64_t processor, uint64_t index, uint64_t content);

uint64_t rvv_get_vlen(uint64_t processor);
uint64_t rvv_get_elen(uint64_t processor);
uint64_t rvv_get_vl(uint64_t processor);
uint64_t rvv_get_sew(uint64_t processor);
uint64_t rvv_get_vtype(uint64_t processor);
float rvv_get_lmul(uint64_t processor);
uint64_t rvv_get_vill(uint64_t processor);

/**
 * addr: the address of memory. The memory is in environment of risc-v execution.
 * len: the length of memory
 * bytes: the buffer to hold the content loaded from memory. It should have same length with `len`
 * note: the memory should be in range [4096, 4096+mem_size]. `mem_size` is specified in rvv_new_processor
 */
int rvv_load_mem(uint64_t processor, uint64_t addr, uint64_t len, uint8_t *bytes);
int rvv_store_mem(uint64_t processor, uint64_t addr, uint64_t len, uint8_t *bytes);

uint64_t rvv_new_disassembler(uint32_t xlen);
int rvv_disassemble(uint64_t dis, uint64_t inst, char *output, uint32_t *output_len);
void rvv_delete_disassembler(uint64_t dis);

void rvv_delete_processor(uint64_t);

#ifdef __cplusplus
}
#endif

#endif // __SPIKE_INTERFCES_H__
