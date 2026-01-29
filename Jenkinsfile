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
						echo "Building Docker image..."
						sh "docker build -t ${DOCKER_IMAGE}:${DOCKER_TAG} ."
					}
				}
			}
		}
		stage('Push Docker Image') {
			steps {
				container('docker') {
					echo "Pushing Docker image to Docker Hub..."
					withCredentials([usernamePassword(credentialsId: 'dockerhub', usernameVariable: 'DOCKERHUB_USER', passwordVariable: 'DOCKERHUB_PASS')]) {
						sh """
						echo \$DOCKERHUB_PASS | docker login -u \$DOCKERHUB_USER --password-stdin
						docker tag ${DOCKER_IMAGE}:${DOCKER_TAG} \$DOCKERHUB_USER/${DOCKER_IMAGE}:${DOCKER_TAG}
						docker push \$DOCKERHUB_USER/${DOCKER_IMAGE}:${DOCKER_TAG}
					"""
					}
				}
			}
		}
		stage('Deploy to K8s') {
			steps {
				container('kubectl') {
					withCredentials([usernamePassword(credentialsId: 'dockerhub', usernameVariable: 'DOCKERHUB_USER', passwordVariable: 'DOCKERHUB_PASS')]) {
						script {
							echo "Deploying to Kubernetes..."
							sh "kubectl create namespace mega-uploader --dry-run=client -o yaml | kubectl apply -f -"

							sh """
							kubectl create configmap mega-uploader-config \
								--namespace mega-uploader \
								--from-literal=COGNITO_CLIENT_ID=5pgshmijcotuq03ur8d3sa73uk \
								--from-literal=COGNITO_DOMAIN=https://mega-upload-saas.auth.us-east-1.amazoncognito.com \
								--from-literal=COGNITO_REDIRECT_URI=http://localhost:8080/auth/cli/callback \
								--from-literal=COGNITO_REGION=us-east-1 \
								--from-literal=COGNITO_USER_POOL_ID=us-east-1_MuSEs6gb1 \
								--from-literal=SERVER_ADDR=0.0.0.0:8080 \
								--from-literal=STS_EXTERNAL_ID=dpaas-cli-auth \
								--from-literal=STS_ROLE_ARN=arn:aws:iam::318361291322:role/server-mega-upload-assumable-role \
								--dry-run=client -o yaml | kubectl apply -f -
							"""

							sh """
							cat <<EOF | kubectl apply -f -
							apiVersion: apps/v1
							kind: Deployment
							metadata:
							  name: mega-uploader-auth
							  namespace: mega-uploader
							spec:
							  replicas: 1
							  selector:
								matchLabels:
								  app: mega-uploader-auth
							  template:
								metadata:
								  labels:
									app: mega-uploader-auth
								spec:
								  containers:
								  - name: mega-uploader-auth
									image: \${DOCKERHUB_USER}/${DOCKER_IMAGE}:${DOCKER_TAG}
									ports:
									- containerPort: 8080
									envFrom:
									- configMapRef:
										name: mega-uploader-config
							EOF
														"""

							sh """
														cat <<EOF | kubectl apply -f -
							apiVersion: v1
							kind: Service
							metadata:
							  name: mega-uploader-auth-svc
							  namespace: mega-uploader
							spec:
							  selector:
								app: mega-uploader-auth
							  ports:
								- protocol: TCP
								  port: 80
								  targetPort: 8080
							EOF
														"""
						}
					}
				}
			}
		}
	}
}