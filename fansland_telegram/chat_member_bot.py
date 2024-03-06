#!/usr/bin/env python
# pylint: disable=unused-argument
# This program is dedicated to the public domain under the CC0 license.

"""
Simple Bot to handle '(my_)chat_member' updates.
Greets new users & keeps track of which chats the bot is in.

Usage:
Press Ctrl-C on the command line or send a signal to the process to stop the
bot.
"""

import logging
from typing import Optional, Tuple

from telegram import Chat, ChatMember, ChatMemberUpdated, Update
from telegram.constants import ParseMode
from telegram.ext import (
    Application,
    ChatMemberHandler,
    CommandHandler,
    ContextTypes,
    MessageHandler,
    filters,
)

# Enable logging
logging.basicConfig(
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s", level=logging.INFO
)

# set higher logging level for httpx to avoid all GET and POST requests being logged
logging.getLogger("httpx").setLevel(logging.WARNING)

logger = logging.getLogger(__name__)


# 处理新用户加入的函数
async def handle_new_member(update: Update, context: ContextTypes.DEFAULT_TYPE):
    print(f"update:{update}")
    new_members = update.message.new_chat_members
    print(f'new_members:{new_members}')
    for member in new_members:
        user_id = member.id
        username = member.username
        first_name = member.first_name
        last_name = member.last_name
        # 处理新加入的用户信息
        print(f"New user joined: ID={user_id}, Username={username}, First Name={first_name}, Last Name={last_name}")

async def hello(update: Update, context: ContextTypes.DEFAULT_TYPE):
    await update.message.reply_text(f'Hello {update.effective_user.first_name}')


def main():
    """Start the bot."""
    # Create the Application and pass it your bot's token.
    application = Application.builder().token("6709002095:AAG3zZpWpwTW_0sT1rscy2Li9C-LBnC6RO8").build()

    # ok , 监听所有更新事件
    # application.add_handler(MessageHandler(filters.StatusUpdate.ALL, handle_new_member))

    # 仅监听新用户加入
    application.add_handler(MessageHandler(filters.StatusUpdate.NEW_CHAT_MEMBERS, handle_new_member))

    application.add_handler(CommandHandler("hello", hello))

    application.run_polling(allowed_updates=Update.ALL_TYPES)


if __name__ == "__main__":
    main()