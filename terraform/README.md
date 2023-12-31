意図的にterraformのs3 backendのbucket とkeyを削除してからgitに入れているので、適宜追加するかbackend自体を削除するかしないと動かない。


secrets/
このプロジェクトのterraformやlambda内で用いる機密情報を格納する場所を管理する。本来はプログラム用とterraform用は分けるべきだが、面倒だったので同一のものにしている。
AWS secrets managerを使う。

    terraform plan -var-file=private.tfvars

infra/
このプロジェクト自体が動く場所を管理する。secretsに依存する。
AWS lambda functionをAWS EventBridgeで定期実行する。
