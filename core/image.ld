/* Copyright (C) 2017 Daniel Page <csdsp@bristol.ac.uk>
 *
 * Use of this source code is restricted per the CC BY-NC-ND license, a copy of 
 * which can be found via http://creativecommons.org (and should be included as 
 * LICENSE.txt within the associated archive or repository).
 */

SECTIONS {
  /* assign load address (per  QEMU) */
  .       =     0x70010000; 
  /* place text segment(s)           */
  .text : { kernel/lolevel.o(.text) *(.text .rodata) }
  /* place data segment(s)           */        
  .data : {                         *(.data        ) }
  /* place bss  segment(s)           */        
  .bss  : {                         *(.bss         ) }
  /* create a heap for malloc (lab4) */
  .heap : {
        end    = .;
    _heap_start = .;
    .             = . + 0x00001000;
    _heap_end    = .;
  }
  /* required for something in libgcc called by Rust alloc */
  .ARM.exidx : {
      __exidx_start = .;
      *(.ARM.exidx* .gnu.linkonce.armexidx.*)
      __exidx_end = .;
  }
  /* align       address (per AAPCS) */
  .       = ALIGN( 8 );        
  /* allocate stack for SVC/IRQ mode     */
  .       = . + 0x00001000;  
  tos_int = .;
  /* allocate stack for P1           */
  .       = . + 0x00001000;
  tos_P1  = .;
  /* allocate stack for P2           */
  .       = . + 0x00001000;
  tos_P2  = .;
}
