pipeline {
    agent { docker { image 'magnonellie/amethyst-dependencies:stable' } }
    stages {
        stage('build') {
            steps {
                sh 'cargo build --all'
            }
        }
    }
}
