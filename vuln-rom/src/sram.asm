SECTION "Save data", SRAM
saveHeader::
ds $10
.end::
playerNameLength::
db
playerName::
ds $8
flagSramLength::
db
flagSram::
ds $20
sramEnd::