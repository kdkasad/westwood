local cwd = vim.fn.getcwd()
if cwd:find("westwood$") then
    -- Include our injection queries
	vim.opt.runtimepath:prepend(cwd .. "/.nvim")
    -- Disable LSP string highlighting because it doesn't support nested injections
    vim.api.nvim_set_hl(0, "@lsp.type.string", {})
end
