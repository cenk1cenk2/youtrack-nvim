local M = {}

---
---@param scratch boolean
---@param modifiable boolean
---@return integer
function M.create_buffer(scratch, modifiable)
	local bufnr = vim.api.nvim_create_buf(false, scratch)

	vim.api.nvim_set_option_value("bufhidden", "wipe", { buf = bufnr })
	vim.api.nvim_set_option_value("modifiable", modifiable, { buf = bufnr })

	return bufnr
end

---
---@param component any
---@param value? any
---@return any
function M.set_component_value(component, value)
	vim.schedule(function()
		if not value then
			value = component:get_current_value()
		end

		component:set_current_value(value)
		if type(component.get_lines) == "function" then
			local lines = component:get_lines()
			vim.api.nvim_buf_set_lines(component.bufnr, 0, -1, true, lines)
		end
		component:redraw()
	end)

	return component
end

---@param bufnr integer
---@return string[] | nil
function M.get_buffer_content(bufnr)
	local content = vim.api.nvim_buf_get_lines(bufnr, 0, -1, false)

	if #content == 0 or (#content == 1 and content[1] == "") then
		return nil
	end

	return content
end

---@param component any
---@return string[] | nil
function M.get_component_buffer_content(component)
	return M.get_buffer_content(component.bufnr)
end

---
---@param component any
---@param content string | string[]
---@return any
function M.set_component_buffer_content(component, content)
	if component.bufnr == nil then
		return component
	end

	---@type string[]
	local c
	if type(content) == "string" then
		c = vim.fn.split(content, "\n")
	elseif type(content) == "table" then
		c = content
	else
		c = { "" }
	end

	local modifiable = vim.api.nvim_get_option_value("modifiable", { buf = component.bufnr })
	if not modifiable then
		vim.api.nvim_set_option_value("modifiable", true, { buf = component.bufnr })
	end

	vim.api.nvim_buf_set_lines(component.bufnr, 0, -1, false, c)
	vim.api.nvim_set_option_value("modified", false, { buf = component.bufnr })
	vim.api.nvim_set_option_value("modifiable", modifiable, { buf = component.bufnr })

	return component
end

---@param renderer any
function M.attach_autoclose(renderer)
	local popups = renderer._private.flatten_tree
	for _, popup in pairs(popups) do
		popup:on("BufLeave", function()
			vim.schedule(function()
				local bufnr = vim.api.nvim_get_current_buf()
				for _, p in pairs(popups) do
					if p.bufnr == bufnr then
						return
					end
				end
				renderer:close()
			end)
		end)
	end
end

---Calculate the size of the UI.
---@param size youtrack.ConfigUiSize
---@return youtrack.ConfigUiSize
function M.renderer_calculate_size(size)
	---@type youtrack.ConfigUiSize
	local result = {}

	if type(size.width) == "number" and size.width <= 1 and size.width > 0 then
		result.width = math.floor(vim.o.columns * size.width)
	elseif type(size.width) == "function" then
		result.width = size.width(vim.o.columns)
	end

	if type(size.height) == "number" and size.height <= 1 and size.height > 0 then
		result.height = math.floor(vim.o.lines * size.height)
	elseif type(size.height) == "function" then
		result.height = size.height(vim.o.rows)
	end

	return result
end

return M
