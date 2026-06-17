-- expect: pass
-- category: tables
-- protected-string: COMMERCIAL_TABLE_KEY_005

local t = { ['COMMERCIAL_TABLE_KEY_005'] = 5, plain = 6 }
print('tables_005', t['COMMERCIAL_TABLE_KEY_005'], t.plain)
