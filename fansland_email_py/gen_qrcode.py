
import qrcode

from qrcode import QRCode
from qrcode.image import pure
import io



def gen_qrcode_png_bytes(content: str):
    qr = qrcode.QRCode(
        version=2,
        error_correction=qrcode.constants.ERROR_CORRECT_L,
        box_size=5,
        border=1
    )#设置二维码的大小
    qr.add_data(content)
    qr.make(fit=True)
    img = qr.make_image(image_factory=pure.PyPNGImage)
    # line2=line[-16:]#取后几位命名
    # print(line2)
    output = io.BytesIO()
    img.save(output)
    img_bytes = output.getvalue()
    # print(img_bytes)
    # xx = open(line2+".png", 'rb').read()
    # print(img_bytes == xx)
    return img_bytes


# gen_qrcode_png_bytes('1:12342340023423432423423423423')
