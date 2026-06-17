-- expect: pass
-- category: tables
-- protected-string: COMMERCIAL_TABLE_KEY_015

local t = { ['COMMERCIAL_TABLE_KEY_015'] = 15, plain = 16 }
print('tables_015', t['COMMERCIAL_TABLE_KEY_015'], t.plain)
