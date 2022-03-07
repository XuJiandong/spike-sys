#ifndef __SPIKE_INTERFCES_H__
#define __SPIKE_INTERFCES_H__

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

uint64_t rvv_new_processor(void);
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
uint64_t rvv_get_lmul(uint64_t processor);
uint64_t rvv_get_vill(uint64_t processor);

void rvv_delete_processor(uint64_t);

#ifdef __cplusplus
}
#endif

#endif // __SPIKE_INTERFCES_H__
