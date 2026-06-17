-- expect: pass
-- category: tables
-- protected-string: COMMERCIAL_TABLE_KEY_010

local t = { ['COMMERCIAL_TABLE_KEY_010'] = 10, plain = 11 }
print('tables_010', t['COMMERCIAL_TABLE_KEY_010'], t.plain)
