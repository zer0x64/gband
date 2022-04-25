SECTION "Save data", SRAM
saveHeader::
ds $10
.end::
saveIsInitialized::
db
playerNameLengthSram::
db
playerNameSram::
ds $8
flagLengthSram::
db
flagSram::
ds $10
sramEnd::