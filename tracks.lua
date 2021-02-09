--- -*- compile-command: "wrk -c4 -t4 -d4s -s tracks.lua http://127.0.0.1:8080"; -*-
wrk.method  = "POST"
wrk.path    = "/tracks"
wrk.body    = "{\"offset\":0,\"limit\":1000}"
wrk.headers["Content-Type"] = "application/json;charset=utf-8"

function response(status, headers, body)
   len = string.len(body)
   if status == 200 and len ~= 175194 then
      print("invalid response length:", len, status)
   end
end
