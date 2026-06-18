local OP_LOADK=101
local OP_CALLGLOBAL=202
local function _f_decode(T) return T end
local function _f_pack(W)
 local O,A,B,C={},{},{},{}
 local j=1
 for i=1,#W,4 do O[j]=W[i]; A[j]=W[i+1]; B[j]=W[i+2]; C[j]=W[i+3]; j=j+1 end
 return {W[1],W[2],W[3],W[4]}
end
local function _f_run()
 error("ferret vm fault")
end
local cache=C[1][C[2][1]]
print("secret_literal")
