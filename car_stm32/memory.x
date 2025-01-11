/* Linker script for STM32F401RE */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 512K
  RAM : ORIGIN = 0x20000000, LENGTH = 96K
}

/* Define the entry point */
ENTRY(Reset);