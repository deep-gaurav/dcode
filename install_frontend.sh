bash /rustinstaller -y
cd /
git clone https://github.com/deep-gaurav/dcodefront.git
cd dcodefront
yarn
PARCEL_WORKERS=1 yarn build
