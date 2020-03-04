/* Copyright (C) 2017 Daniel Page <csdsp@bristol.ac.uk>
 *
 * Use of this source code is restricted per the CC BY-NC-ND license, a copy of 
 * which can be found via http://creativecommons.org (and should be included as 
 * LICENSE.txt within the associated archive or repository).
 */

#ifndef __HILEVEL_H
#define __HILEVEL_H

// Include functionality relating to newlib (the standard C library).

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// Include functionality relating to the platform.

#include   "GIC.h"
#include "PL011.h"

// Include functionality relating to the   kernel.

#include "lolevel.h"
#include     "int.h"

typedef struct {
    uint32_t cpsr, pc, gpr[ 13 ], sp, lr;
} ctx_t;

void hilevel_handler_rst_c(ctx_t* ctx);
void hilevel_handler_irq_c();
void hilevel_handler_svc_c(ctx_t* ctx, uint32_t id);

#endif
