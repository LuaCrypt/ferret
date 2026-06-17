local n = 0
goto enter

::again::
n = n + 1

::enter::
if n < 4 then
    goto again
end

print("conf_goto", n)
