--- Promise.lua - an implementation of JavaScript-style promises in Lua
-- @author [Ezhik](https://ezhik.jp)
-- @module Promise
-- @license MPL-2.0
-- @version 1.1.0

-- #region Core implementation
---@class Promise
---@field private _internal PromiseInternal
local Promise = {}
Promise.__index = Promise
Promise.__name = "Promise"

---@alias PromiseState
---| 0 # PENDING
---| 1 # FULFILLED
---| 2 # REJECTED
local PENDING = 0
local FULFILLED = 1
local REJECTED = 2

---@type table<PromiseState, string>
local stateNames = {
    [PENDING] = "pending",
    [FULFILLED] = "fulfilled",
    [REJECTED] = "rejected",
}

function Promise:__tostring()
    local x = self._internal
    return string.format("Promise {<%s>}", stateNames[x.state])
end

---Function which is used to schedule the execution of a callback asynchronously (hopefully)
---Override this to use a different scheduler
---@param fn function The function to be scheduled
---@return nil
function Promise.schedule(fn)
    pcall(fn)
end

---Check if the given value is callable
---@param fn any
---@return boolean
local function isCallable(fn)
    if type(fn) == "function" then
        return true
    elseif type(fn) == "table" then
        local mt = getmetatable(fn)
        return mt and mt.__call
    end
    return false
end

---@class PromiseInternal
---@field value any[]?
---@field state PromiseState
---@field onFulfilled function[]
---@field onRejected function[]


---@alias resolve fun(...) - resolve supports multiple return args like lua returns
---@alias reject fun(any) - reject supports only one like lua error()

---Creates a new Promise
---@param fn fun(resolve: resolve, reject: reject) Function which will resolve or reject this promise
---@return Promise
function Promise.new(fn)
    local self = setmetatable({
        _internal = {
            value = nil,
            state = PENDING,
            onFulfilled = {},
            onRejected = {},
        },
    }, Promise)
    local x = self._internal

    local function triggerPromiseReactions(reactions, value)
        for _, reaction in ipairs(reactions) do
            local job = function() reaction(table.unpack(value)) end
            Promise.schedule(job)
        end
    end

    local function fulfillPromise(...)
        local reactions = x.onFulfilled
        x.onFulfilled = nil
        x.onRejected = nil
        x.state = FULFILLED
        x.value = { ... }
        triggerPromiseReactions(reactions, x.value)
    end

    local function rejectPromise(reason)
        local reactions = x.onRejected
        x.onFulfilled = nil
        x.onRejected = nil
        x.state = REJECTED
        x.value = { reason }
        triggerPromiseReactions(reactions, x.value)
    end

    -- CreateResolvingFunctions (ECMA-262 27.2.1.3)
    local function createResolvingFunctions()
        local alreadyResolved = false
        local resolve, reject

        resolve = function(resolution, ...)
            if alreadyResolved then return end
            alreadyResolved = true

            if resolution == self then
                rejectPromise("TypeError: Promise resolved with itself")
                return
            end

            if type(resolution) ~= "table" then
                fulfillPromise(resolution, ...)
                return
            end

            local ok, next = pcall(function() return resolution.next end)
            if not ok then
                rejectPromise(next)
                return
            end

            if not isCallable(next) then
                fulfillPromise(resolution, ...)
                return
            end

            -- NewPromiseResolveThenableJob (ECMA-262 27.2.2.2)
            Promise.schedule(function()
                local thenableResolve, thenableReject = createResolvingFunctions()
                local ok, err = pcall(next, resolution, thenableResolve, thenableReject)
                if not ok then
                    thenableReject(err)
                end
            end)
        end

        reject = function(reason)
            if alreadyResolved then return end
            alreadyResolved = true
            rejectPromise(reason)
        end

        return resolve, reject
    end

    local resolve, reject = createResolvingFunctions()

    local ok, err = pcall(fn, resolve, reject)
    if not ok then
        reject(err)
    end

    return self
end

---@alias PromiseCallback fun(...): ...

local function handleTry(resolve, reject, ok, ...)
    if ok then
        resolve(...)
    else
        reject(...)
    end
end

---Registers callbacks for the resolution or rejection of the promise
---@param onFulfilled? PromiseCallback
---@param onRejected? PromiseCallback
---@return Promise
function Promise:next(onFulfilled, onRejected)
    local x = self._internal
    return Promise.new(function(resolve, reject)
        if x.state == FULFILLED then
            Promise.schedule(function()
                if onFulfilled and isCallable(onFulfilled) then
                    handleTry(resolve, reject, pcall(onFulfilled, table.unpack(x.value)))
                else
                    resolve(table.unpack(x.value))
                end
            end)
        elseif x.state == REJECTED then
            Promise.schedule(function()
                if onRejected and isCallable(onRejected) then
                    handleTry(resolve, reject, pcall(onRejected, table.unpack(x.value)))
                else
                    reject(table.unpack(x.value))
                end
            end)
        else
            if onFulfilled and isCallable(onFulfilled) then
                table.insert(x.onFulfilled, function(...) handleTry(resolve, reject, pcall(onFulfilled, ...)) end)
            else
                table.insert(x.onFulfilled, resolve)
            end

            if onRejected and isCallable(onRejected) then
                table.insert(x.onRejected, function(...) handleTry(resolve, reject, pcall(onRejected, ...)) end)
            else
                table.insert(x.onRejected, reject)
            end
        end
    end)
end

-- #endregion


-- #region Promise shortcuts

---Registers a callback only for the rejection of a promise
---@param onRejected? PromiseCallback
---@return Promise
function Promise:catch(onRejected)
    return self:next(nil, onRejected)
end

---Registers a callback that will be called when the promise is settled
---The resolved value cannot be modified from the callback
---@param onFinally? fun(): any
---@return Promise
function Promise:finally(onFinally)
    return self:next(
        function(...)
            if not (onFinally and isCallable(onFinally)) then
                return ...
            end
            local args = { ... }
            return Promise.resolve(onFinally()):next(function()
                return table.unpack(args)
            end)
        end,
        function(reason)
            if not (onFinally and isCallable(onFinally)) then
                error(reason)
            end
            return Promise.resolve(onFinally()):next(function()
                error(reason)
            end)
        end
    )
end

---Creates a promise resolved with the given value
---@param value any
---@return Promise
function Promise.resolve(value, ...)
    if select("#", ...) == 0 and getmetatable(value) == Promise then
        return value
    else
        local promise, resolve, reject = Promise.withResolvers()
        resolve(value, ...)
        return promise
    end
end

---Creates a promise rejected with the given reason
---@param reason any
---@return Promise
function Promise.reject(reason)
    local promise, resolve, reject = Promise.withResolvers()
    reject(reason)
    return promise
end

---Creates a new promise and returns it along with its resolve and reject functions
---@return Promise, resolve, reject
function Promise.withResolvers()
    local resolve, reject
    local promise = Promise.new(function(resolve_, reject_)
        resolve = resolve_
        reject = reject_
    end)

    return promise, resolve, reject
end

---Takes a callback and wraps its result in a promise
---@param fn fun(...): any A callback
---@param ... unknown Arguments to pass to the callback
---@return Promise
function Promise.try(fn, ...)
    local promise, resolve, reject = Promise.withResolvers()
    handleTry(resolve, reject, pcall(fn, ...))
    return promise
end

-- #endregion


-- #region Promise combinators

---Returns a promise that is resolved with an array of results when all of the provided promises resolve
---or rejected when any promise is rejected
---Note that only the first return value of the result will be provided to be consistent with JavaScript.
---@param promises any[]
---@return Promise
function Promise.all(promises)
    return Promise.new(function(resolve, reject)
        local results = {}
        local remaining = #promises

        if remaining == 0 then
            resolve(results)
            return
        end

        for i, value in ipairs(promises) do
            Promise.resolve(value):next(function(value)
                results[i] = value
                remaining = remaining - 1
                if remaining == 0 then
                    resolve(results)
                end
            end, reject)
        end
    end)
end

---Returns a promise that is resolved or rejected when any of the provided promises resolve or reject
---@param promises any[]
---@return Promise
function Promise.race(promises)
    return Promise.new(function(resolve, reject)
        for _, value in ipairs(promises) do
            Promise.resolve(value):next(resolve, reject)
        end
    end)
end

---Returns a promise that is resolved when any of the provided promises resolve or rejected when all of the provided promises reject
---@param promises any[]
---@return Promise
function Promise.any(promises)
    return Promise.new(function(resolve, reject)
        local errors = {}
        local remaining = #promises

        if remaining == 0 then
            reject({ "No promises were resolved" })
            return
        end

        for i, value in ipairs(promises) do
            Promise.resolve(value):next(resolve, function(reason)
                errors[i] = reason
                remaining = remaining - 1
                if remaining == 0 then
                    reject(errors)
                end
            end)
        end
    end)
end

---Returns a promise that is resolved with an array of results when all of the provided promises resolve or reject
---Note that only the first return value of the result will be provided to be consistent with JavaScript.
---@param promises any[]
---@return Promise
function Promise.allSettled(promises)
    return Promise.new(function(resolve)
        local results = {}
        local remaining = #promises

        if remaining == 0 then
            resolve(results)
            return
        end

        for i, value in ipairs(promises) do
            Promise.resolve(value):next(
                function(value) results[i] = { status = "fulfilled", value = value } end,
                function(reason) results[i] = { status = "rejected", reason = reason } end
            ):finally(function()
                remaining = remaining - 1
                if remaining == 0 then
                    resolve(results)
                end
            end)
        end
    end)
end

-- #endregion

-- #region Extensions

-- #region Rust-ish
---Returns a promise that will cast rejections to nil
---@return Promise
function Promise:ok()
    return self:next(nil, function() end)
end

-- #endregion

-- #region For debugging
---Will print the resolution result to the console when the promise is resolved
---@return Promise
function Promise:print()
    return self:next(
        function(...)
            print("Fulfilled:", ...)
            return ...
        end,
        function(reason)
            print("Rejected:", reason)
            return reason
        end
    )
end

-- #endregion

-- #region async-await
---If executed from a coroutine, will yield until the promise is resolved and return the resolved value
---@return any ...
function Promise:await()
    if getmetatable(self) ~= Promise then
        return Promise.resolve(self):await()
    end

    if not coroutine.isyieldable() then
        error("Promise:await() must be called from a yieldable coroutine")
    end

    local x = self._internal

    if x.state == PENDING then
        local co = coroutine.running()
        self:finally(function()
            coroutine.resume(co)
        end)
        coroutine.yield()
    end

    if x.state == FULFILLED then
        return table.unpack(x.value)
    elseif x.state == REJECTED then
        error(x.value[1])
    else
        Promise.thisIsNeverSupposedToHappen = self
        error("Promise is in an invalid state")
        print("Promise is in an invalid state, check Promise.thisIsNeverSupposedToHappen")
    end
end

local asyncFnCache = setmetatable({}, { __mode = "k" }) -- weak keys so we don't cache functions that no longer exist
---Wraps potentially asynchronous functions to make them return a promise
---@param fn fun(...): ... A function to wrap
---@return fun(...): Promise A wrapped function
function Promise.async(fn)
    if not asyncFnCache[fn] then
        asyncFnCache[fn] = function(...)
            local args = { ... }
            return Promise.new(function(resolve, reject)
                local co = coroutine.create(function()
                    handleTry(resolve, reject, pcall(fn, table.unpack(args)))
                end)
                Promise.schedule(function()
                    coroutine.resume(co)
                end)
            end)
        end
        -- map the new async function to itself so that async(async(fn)) == async(fn)
        asyncFnCache[asyncFnCache[fn]] = asyncFnCache[fn]
    end
    return asyncFnCache[fn]
end

-- #endregion

-- #endregion

return Promise
