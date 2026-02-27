# Boards

## **TAG**
STM32U575 (1Mb or higher flash, specifically for dual bank capabilitity in OTA firmware)
- Naming scheme: TMDT1CU1 `TMDT{generation}{battery grade}{sensor grade}{revision}`

### Generation
- Increment on completely new systems

### Battery grades
- C (coin cell, 225mAh)
- M (mid Li-Po, 1500mAh)
- H (high capacity Li-Po cell, 3000mAh)
- X (extended battery, 3500mAh+)

### Sensor grades
- A (Accelerometer)
- G (above + Gyro)
- E (above + Barometer)
- F (full - everything plus any future sensors)

### Revision
- Increment on minor adjustments (antenna updates, component changes, 6 axis -> 9 axis IMU)

## **ANCHOR**
STM32H7R7 (600Mhz, 64Kb of bootflash, external flash options, 620Kb SRAM)

##H **HUB**
STM32N645X0 (800Mhz, 64Kb of bootflash, external flash options, 4.2Mb SRAM)
