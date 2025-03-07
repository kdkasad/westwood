local cwd = vim.fn.getcwd()
if cwd:find("westwood$") then
	vim.opt.runtimepath:prepend(cwd .. "/.nvim")
end
