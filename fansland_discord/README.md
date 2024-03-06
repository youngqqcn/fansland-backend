
## Discord

https://discordpy.readthedocs.io/en/latest/discord.html

Token: MTIxNDQ5ODEyMjY1NTQ3MzcwNA.GfjsqJ.ZxH-rrdlyCetZoQLZ1EFUgvjyeWIcOksMrx9s4

机器人:

### Discord 授权
- 参考： https://github.com/youngqqcn/guide/tree/main/code-samples/oauth/simple-oauth-webserver

### 加入Discord的需求实现

- 方案1：可以通过: https://discordpy.readthedocs.io/en/stable/api.html#members
  - `on_member_join`
  - `on_member_remove`
- 方案2：通过定时任务，定时获取所有members,存入redis
  - https://github.com/Rapptz/discord.py/blob/master/examples/background_task.py


### 邀请好友加入Discord的需求实现方案：
- 参考
  - 邀请追踪：https://github.com/GregTCLTK/Discord-Invite-Tracker
  - https://github.com/GregTCLTK/Discord-Invite-Tracker/blob/7b3f397e26d1953fe3609e5bd72dcfe7849b799f/invite_tracker.py#L58
    - 通过uses来记录邀请成功的人数

### 在Discord中发送消息实现方案：
- 通过on_message, 监听消息，  消息需要加上前缀：
- https://github.com/Rapptz/discord.py/blob/master/examples/reply.py
