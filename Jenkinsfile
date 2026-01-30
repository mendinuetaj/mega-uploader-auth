pipeline {
	agent {
		label 'k8s-agent'
	}
	environment {
		GIT_REPO = 'git@github.com:mendinuetaj/mega-uploader-auth.git'
		GIT_CREDENTIALS = 'github'
		DOCKER_IMAGE = 'mega-uploader-auth'
		DOCKER_TAG = "${env.GIT_TAG?:'latest'}"
	}
	stages {
		stage('Checkout') {
			steps {
				echo "Checking out..."
				checkout([
					$class: 'GitSCM',
					branches: [[name: "*/${env.GIT_TAG?:'master'}"]],
					userRemoteConfigs: [[
						url: env.GIT_REPO,
						credentialsId: env.GIT_CREDENTIALS
					]]
				])
			}
		}
		stage('Build Docker Image') {
			steps {
				container('docker') {
					script {
						echo "Building Docker image for multiple architectures..."
						withCredentials([usernamePassword(credentialsId: 'dockerhub', usernameVariable: 'DOCKERHUB_USER', passwordVariable: 'DOCKERHUB_PASS')]) {
							sh """
							echo \$DOCKERHUB_PASS | docker login -u \$DOCKERHUB_USER --password-stdin
							docker buildx create --use || true
							docker buildx build --platform linux/amd64,linux/arm64 \
								-t \$DOCKERHUB_USER/${DOCKER_IMAGE}:${DOCKER_TAG} --push .
							"""
						}
					}
				}
			}
		}
		stage('Push Docker Image') {
			steps {
				echo "Skipping push as buildx already pushed the multi-arch image"
			}
		}
		stage('Deploy to K8s') {
			steps {
				container('kubectl') {
					withCredentials([usernamePassword(credentialsId: 'dockerhub', usernameVariable: 'DOCKERHUB_USER', passwordVariable: 'DOCKERHUB_PASS')]) {
						withAWS(credentials: 'aws_creds_megaupload_dpaas_account', region: 'us-east-1') {
							script {
								echo "Deploying to Kubernetes..."
								sh "kubectl apply -f k8s/namespace.yaml"
								sh "kubectl apply -f k8s/configmap.yaml"
								sh "envsubst < k8s/aws-secret.yaml | kubectl apply -f -"
								sh "envsubst < k8s/deployment.yaml | kubectl apply -f -"
								sh "kubectl apply -f k8s/service.yaml"
							}
						}
					}
				}
			}
		}
	}
}