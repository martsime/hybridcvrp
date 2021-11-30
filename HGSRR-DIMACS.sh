#!//bin/bash

INSTANCE=$1
ROUNDED=$2
TIME_LIMIT=$3

example () {
    echo "Example: $0 <Instance path> <Rounded> <Time limit>"
}

if [ $INSTANCE = "" ]
then
    echo "ERROR: Instance is not provided!"
    example
    exit 1
fi

if [ $ROUNDED = "" ]
then
    echo "ERROR: Rounded is not provided!"
    example
    exit 1
fi

if [[ $TIME_LIMIT = ""  ||  $TIME_LIMIT -le 0 ]]
then
    echo "ERROR: Time limit is not a positive number!"
    example
    exit 1
fi

if [ $ROUNDED -eq 1 ]
then
    ./hybridcvrp $INSTANCE -t $TIME_LIMIT -r
else
    ./hybridcvrp $INSTANCE -t $TIME_LIMIT
fi
