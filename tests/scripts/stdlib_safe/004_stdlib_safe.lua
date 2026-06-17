-- expect: pass
-- category: stdlib_safe

local t = table.pack('a', 'b', '4')
print('stdlib_004', t.n, table.concat(t, ':'))
