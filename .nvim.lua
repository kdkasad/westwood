-- Load custom injection queries
local cwd = vim.fn.getcwd()
if cwd:find("westwood$") then
    -- Include our injection queries
    vim.opt.runtimepath:prepend(cwd .. "/.nvim")
    -- Disable LSP string highlighting because it doesn't support nested injections
    vim.api.nvim_set_hl(0, "@lsp.type.string", {})
end

-- Make gd jump to rule definitions in grammar.js
vim.api.nvim_create_autocmd({ "BufRead", "BufNew" }, {
    pattern = "grammar.js",
    group = vim.api.nvim_create_augroup("westwood", {}),
    callback = function()
        vim.api.nvim_buf_set_keymap(0, "n", "gd", "", {
            callback = function()
                -- Search for the word under the cursor followed by a colon
                vim.fn.search("\\<" .. vim.fn.expand("<cword>") .. "\\>:", "scw")
            end,
        })
    end,
})
