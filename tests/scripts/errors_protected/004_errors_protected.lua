-- expect: pass
-- category: errors_protected

local ok = pcall(function()
    error('err_004')
end)
print('errors_004', ok)
