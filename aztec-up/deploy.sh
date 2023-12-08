set -e

BRANCH=$1

export TF_VAR_BRANCH=$BRANCH

# Downloads and installs `terraform` if it's not installed.
if [ ! -f /usr/local/bin/terraform ]; then
  cd $HOME
  TERRAFORM_VERSION=1.5.2
  curl -sSL https://releases.hashicorp.com/terraform/${TERRAFORM_VERSION}/terraform_${TERRAFORM_VERSION}_linux_amd64.zip -o terraform.zip
  sudo apt install -y unzip
  unzip terraform.zip
  sudo mv terraform /usr/local/bin/
  rm terraform.zip
  cd -
fi

echo "Initializing terraform"
terraform init -input=false -backend-config="key=aztec-sandbox-website/$BRANCH"

echo "Applying terraform config"
terraform apply -input=false -auto-approve