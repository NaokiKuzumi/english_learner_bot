#
# Secret manager used for the main infra secrets.
#
variable "terraform_apply_role" {
  description = "Assuming Role ARN that applies this terraform."
  type        = string
  sensitive = true
}



terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.30"
    }
  }
  # Hardcoded because terraform doesn't allow vars here...
  backend "s3" {
    bucket =
    key    =
    region = "ap-northeast-1"
  }

  required_version = ">= 0.14.9"
}



provider "aws" {
  profile = "default"
  region  = "ap-northeast-1"

  default_tags {
    tags = {
      project = "english_leaner_bot"
    }
  }

  assume_role {
    role_arn = var.terraform_apply_role
  }
}

resource "aws_secretsmanager_secret" "english_learner_bot_secrets" {
  name = "english_learner_bot_secret"
}




