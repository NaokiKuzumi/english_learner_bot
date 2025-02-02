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

data "aws_secretsmanager_secret" "secrets" {
  name = "english_learner_bot_secret"
}

data "aws_secretsmanager_secret_version" "main_secrets" {
  secret_id = data.aws_secretsmanager_secret.secrets.id
}

//
// Some secrets that not managed by this terraform and not included in the repository.
//
locals {
  decoded_secrets = jsondecode(data.aws_secretsmanager_secret_version.main_secrets.secret_string)
  lambda_executor_role = local.decoded_secrets["lambda_executor_role"]
  scheduler_executor_role = local.decoded_secrets["scheduler_executor_role"]
}

output "lambda_role" {
  sensitive = true
  value = local.lambda_executor_role
}

output "scheduler_role" {
  sensitive = true
  value = local.scheduler_executor_role
}

resource "aws_lambda_function" "english_learner_bot_lambda" {
  filename = "../../bot_lambda/target/lambda/src/bootstrap.zip"
  source_code_hash = filebase64sha256("../../bot_lambda/target/lambda/src/bootstrap.zip")
  function_name = "english_learner_bot"
  handler = "rust.handler"
  runtime = "provided.al2023"
  role = local.lambda_executor_role
  architectures = ["arm64"]
}

// https://docs.aws.amazon.com/scheduler/latest/UserGuide/schedule-types.html
resource "aws_scheduler_schedule" "english_learner_cron" {
  name = "english_learner_cron"
  schedule_expression = "rate(60 minutes)"
  description = "Schedule the invoke of english_learner_bot. One run is posting one truth."

  flexible_time_window {
    mode = "FLEXIBLE"
    maximum_window_in_minutes = 5
  }

  target {
    arn      = aws_lambda_function.english_learner_bot_lambda.arn
    role_arn = local.scheduler_executor_role
    input = "{}"
  }
}
