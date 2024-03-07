from telegram import Update
from telegram.ext import ApplicationBuilder, CommandHandler, ContextTypes


async def hello(update: Update, context: ContextTypes.DEFAULT_TYPE) -> None:
    await update.message.reply_text(f'Hello {update.effective_user.first_name}')


# 定义/start命令的处理函数
async def start(update, context):
    user_id = update.message.from_user.id
    # 解析/start命令后的参数作为邀请人的唯一标识
    args = context.args
    if len(args) > 0:
        inviter_id = args[0]
        # 在这里记录邀请信息，例如更新数据库
        print(f"用户 {user_id} 被 {inviter_id} 邀请。")
        # 没有参数的情况，普通的/start命令处理
        await update.message.reply_text(f"用户 {user_id} 被 {inviter_id} 邀请。")
    else:
        # 没有参数的情况，普通的/start命令处理
        await update.message.reply_text('欢迎使用我们的Bot！')


app = ApplicationBuilder().token("6709002095:AAG3zZpWpwTW_0sT1rscy2Li9C-LBnC6RO8").build()

app.add_handler(CommandHandler("hello", hello))
app.add_handler(CommandHandler('start', start))


app.run_polling()