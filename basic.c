#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "ds.h"
#include "partition.h"
#include "fstest1.h"

int main()
{
    FILE *f,*fp;
    char name[32],*command,p[10]="test1",path[10],path2[10],path3[10],path4[10];
    ffolder root;
    freesec k;
    char i=0;

    printf("=================================================\n");
    printf("   PSLFS - Virtual File System Initialization   \n");
    printf("=================================================\n\n");

    strcpy(path,p);
    strcat(path,".psl");

    strcpy(path2,p);
    strcat(path2,".fol");
    strcpy(path3,p);
    strcat(path3,".fil");
    strcpy(path4,p);
    strcat(path4,".fs");

    printf("Creating partition files...\n");

    f=fopen(path,"wb");
    if(f == NULL) {
        printf("Error: Cannot create partition file %s\n", path);
        return 1;
    }

    strcpy(root.name,"pslos");
    fwrite(&i,sizeof(i),1,f);
    root.sector=1;
    root.upsector=0;
    root.insector=0;
    root.filesector=0;
    root.next=0;
    root.prev=0;
    root.size=0;
    root.properties[0]='r';
    root.properties[1]='w';
    root.properties[2]='v';
    fwrite(&root,sizeof(root),1,f);
    fclose(f);

    f=fopen(path,"r");
    if(f == NULL) {
        printf("Error: Cannot open partition file %s\n", path);
        return 1;
    }

    fp=fopen(path2,"wb");
    if(fp == NULL) {
        printf("Error: Cannot create folder free sectors file %s\n", path2);
        fclose(f);
        return 1;
    }

    k.next=0;
    fseek(f,0,2);
    k.sector=ftell(f);
    fwrite(&k,sizeof(k),1,fp);
    fclose(f);
    fclose(fp);

    fp=fopen(path3,"wb");
    if(fp == NULL) {
        printf("Error: Cannot create file free sectors file %s\n", path3);
        return 1;
    }
    fwrite(&k,sizeof(k),1,fp);
    fclose(fp);

    fp=fopen(path4,"wb");
    if(fp == NULL) {
        printf("Error: Cannot create file content free sectors file %s\n", path4);
        return 1;
    }
    fwrite(&k,sizeof(k),1,fp);
    fclose(fp);

    printf("Partition files created successfully!\n\n");
    printf("Now creating user account...\n");
    makeUser();

    printf("\nFilesystem initialized successfully!\n");
    printf("Run './fbase' to start using the filesystem.\n\n");

    return 0;
}
