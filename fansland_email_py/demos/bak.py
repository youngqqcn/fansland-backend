import os
import boto3
from botocore.exceptions import ClientError
from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText
from email.mime.image import MIMEImage
from email.mime.application import MIMEApplication
import os
from fansland_email_py.lib.template import get_mint_template

from fansland_email_py.lib.gen_qrcode import gen_qrcode_png_bytes

# Replace sender@example.com with your "From" address.
# This address must be verified with Amazon SES.
SENDER = "Fansland <no-reply@fansland.io>"

# Replace recipient@example.com with a "To" address. If your account
# is still in the sandbox, this address must be verified.
RECIPIENT = "youngqqcn@gmail.com"

# Specify a configuration set. If you do not want to use a configuration
# set, comment the following variable, and the
# ConfigurationSetName=CONFIGURATION_SET argument below.
# CONFIGURATION_SET = "ConfigSet"

# If necessary, replace us-west-2 with the AWS Region you're using for Amazon SES.
AWS_REGION = "ap-northeast-1"

# The subject line for the email.
SUBJECT = "Customer service contact info"

# The full path to the file that will be attached to the email.
ATTACHMENT = "./qrcode.png"

# The email body for recipients with non-HTML email clients.
# BODY_TEXT = "Hello,\r\nPlease see the attached file for a list of customers to contact."



# The character encoding for the email.
CHARSET = "utf-8"

# Create a new SES resource and specify a region.
client = boto3.client('ses',region_name=AWS_REGION,
    aws_access_key_id='AKIAWTXOSMHXSIT5FEFQ',
        aws_secret_access_key='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')

# Create a multipart/mixed parent container.
msg = MIMEMultipart('mixed')
# Add subject, from and to lines.
msg['Subject'] = SUBJECT
msg['From'] = SENDER
msg['To'] = RECIPIENT

# Create a multipart/alternative child container.
msg_body = MIMEMultipart('alternative')

# Encode the text and HTML content and set the character encoding. This step is
# necessary if you're sending a message with characters outside the ASCII range.
# textpart = MIMEText(BODY_TEXT.encode(CHARSET), 'plain', CHARSET)

image_count = 2

htmlpart = MIMEText(get_mint_template("0x51Bdbad59a24207b32237e5c47E866A32a8D5Ed8", image_count).encode(CHARSET), 'html', CHARSET)

# Add the text and HTML parts to the child container.
# msg_body.attach(textpart)
msg_body.attach(htmlpart)

# embed_qrcode = MIMEImage(open(ATTACHMENT, 'rb').read())
for i in range(image_count):
    img_data = gen_qrcode_png_bytes("1:1234567891011123423423423423432423423432index_{}".format(i))
    embed_qrcode = MIMEImage(img_data)
    embed_qrcode.add_header('Content-ID', '<qrcode{}>'.format(i))
    embed_qrcode.add_header('Content-Disposition', 'inline', filename="qrcode{}.png".format(i))
    msg_body.attach(embed_qrcode);

# Define the attachment part and encode it using MIMEApplication.
# att = MIMEApplication(open(ATTACHMENT, 'rb').read())
for i in range(image_count):
    att = MIMEApplication(img_data)
    # Add a header to tell the email client to treat this part as an attachment,
    # and to give the attachment a name.
    att.add_header('Content-Disposition','attachment',filename="qrcode{}.png".format(i))

    # Attach the multipart/alternative child container to the multipart/mixed
    # parent container.
    msg.attach(msg_body)

# Add the attachment to the parent container.
msg.attach(att)
#print(msg)
try:
    #Provide the contents of the email.
    response = client.send_raw_email(
        Source=SENDER,
        Destinations=[
            RECIPIENT
        ],
        RawMessage={
            'Data':msg.as_string(),
        },
        # ConfigurationSetName=CONFIGURATION_SET
    )
# Display an error if something goes wrong.
except ClientError as e:
    print(e.response['Error']['Message'])
else:
    print("Email sent! Message ID:"),
    print(response['MessageId'])