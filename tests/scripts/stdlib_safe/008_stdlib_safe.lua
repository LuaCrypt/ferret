-- expect: pass
-- category: stdlib_safe

local t = table.pack('a', 'b', '8')
print('stdlib_008', t.n, table.concat(t, ':'))
