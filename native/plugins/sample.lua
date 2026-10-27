-- Copy into $XDG_DATA_HOME/observatory-core/plugins/ (normally ~/.local/share/...).
-- Restart or use Reload plugins. This file runs natively inside Observatory.
local rows = {}
local batch = false
local active_notification

local extension = {
    id = "org.observatory.example.journey",
    name = "Journey Log",
    version = "1.0.0",
    description = "Records jumps and demonstrates the native extension API.",
    author = "Observatory contributors",
    settings = { notify_jumps = true },
}

function extension.on_load()
    local launches = core.get_storage("launches", 0) + 1
    core.set_storage("launches", launches)
    core.log("Journey Log loaded " .. launches .. " times")
end

function extension.on_ready()
    core.clear_rows()
end

function extension.on_state_changed(state)
    batch = state.batch
end

function extension.on_journal(event)
    if event.event ~= "FSDJump" then return end
    local row = {
        Time = event.timestamp,
        System = event.StarSystem,
        Distance = event.JumpDist or 0,
    }
    rows[#rows + 1] = row
    core.add_row(row)
    core.set_storage("last_system", event.StarSystem)
    core.send_message({ kind = "arrival", system = event.StarSystem })
    if extension.settings.notify_jumps and not batch then
        if active_notification then core.cancel_notification(active_notification) end
        active_notification = core.notify({
            timestamp = event.timestamp,
            system = event.StarSystem,
            title = "Arrived at " .. event.StarSystem,
            detail = string.format("Jumped %.1f light years", event.JumpDist or 0),
            timeout = 8000,
            speech = false,
        })
    end
end

function extension.on_message(message)
    if message.data.kind == "request_last_system" then
        -- message.source contains the host-supplied name, id and version.
        core.log(message.source.name .. " requested the last system: " ..
            core.get_storage("last_system", "unknown"))
    end
end

local function quote(value, delimiter)
    value = tostring(value or "")
    if value:find(delimiter, 1, true) or value:find('["\r\n]') then
        return '"' .. value:gsub('"', '""') .. '"'
    end
    return value
end

function extension.on_export(delimiter)
    local output = { table.concat({"Time", "System", "Distance"}, delimiter) }
    for _, row in ipairs(rows) do
        output[#output + 1] = table.concat({quote(row.Time, delimiter),
            quote(row.System, delimiter), quote(row.Distance, delimiter)}, delimiter)
    end
    return table.concat(output, "\n") .. "\n"
end

function extension.on_shutdown()
    if active_notification then core.cancel_notification(active_notification) end
end

return extension
